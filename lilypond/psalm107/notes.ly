melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 d2 a'2 d4 d4 c2 a2 r2 \break

  % Line 2
  a2 bes4 a4 f2 g2 f2 r2 \break

  % Line 3
  d2 d2 a'2 d4 d4 c2 a2 r2 \break

  % Line 4
  f2 g4 a4 f2 e2 d2 r2 \break

  % Line 5
  d'2 a4 b4 c2 b2 a2 r2 \break

  % Line 6
  d2 e4 d4 a4 c4 b2 a2 \break

  % Line 7
  a2 c4 b4 a2 g2 f2 r2 \break

  % Line 8
  f2 g4 a4 g4 f4 e2 d1 \bar "|."
}
