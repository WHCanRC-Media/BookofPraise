melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 a2 d2 c4 a4 c4 c4 b4 a4 gis2 a2 r2 \break

  % Line 2
  a2 f4 d4 a'2 a2 c4 b4 a4 d2 cis4 d2 r2 \break

  % Line 3
  a2 bes4 g4 a2 f2 g4 a4 d,4 f4 e2 d2 r2 \break

  % Line 4
  a'2 b4 c4 d2 a2 d4 d4 c4 c4 b2 a2 r2 \break

  % Line 5
  a2 g2 a2 f4 d4 a'4 bes4 a2 g2 f2 r2 \break

  % Line 6
  a2 d2 c2 a4 f4 g4 d4 f2 e2 d1 \bar "|."
}
