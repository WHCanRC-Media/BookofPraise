melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f2 g4 g4 a2 a2 bes4 a4 g4 f4 e2 d2 r2 a'2 f4 g4 a2 c2 c4 c4 b4 a4 gis2 a2 r2 \break

  % Line 2
  d2 c4 b4 a2 d2 e4 c4 d2 b2 a2 r2 \break

  % Line 3
  f2 g4 a4 bes4 a4 g4 f4 g2 e2 d2 r2 \break

  % Line 4
  d2 f4 g4 a2 d2 c4 b4 a4 g4 f2 e2 r2 a2 g4 g4 f4 d4 f4 g4 a4 bes4 g2 f2 r2 \break

  % Line 5
  d2 a'4 a4 g4 f4 e4 d4 f2 g2 a2 r2 \break

  % Line 6
  d2 c4 b4 a2 g2 f4 g4 f2 e2 d1 \bar "|."
}
